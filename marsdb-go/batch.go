package marsdb

// Pure-Go decoder for the marsdb-capi batch format (see marsdb.h's
// "batch lane" spec block — this file must track it byte for byte).
// One cgo crossing fetches the whole result; everything here is
// stdlib-only Go, so the binding keeps zero external dependencies and
// int64 precision is exact by construction (varints, not text).

import (
	"errors"
	"fmt"
	"math"
)

const (
	tagNull     = 0x00
	tagBool     = 0x01
	tagInt      = 0x02
	tagFloat    = 0x03
	tagString   = 0x04
	tagDate     = 0x05
	tagDuration = 0x06
	tagNode     = 0x07
	tagEdge     = 0x08
	tagList     = 0x09
	tagMap      = 0x0a
	tagPath     = 0x0b
)

type batchReader struct {
	buf []byte
	pos int
}

var errBatchTruncated = errors.New("marsdb: truncated batch payload")

func (r *batchReader) u8() (byte, error) {
	if r.pos >= len(r.buf) {
		return 0, errBatchTruncated
	}
	b := r.buf[r.pos]
	r.pos++
	return b, nil
}

func (r *batchReader) varint() (uint64, error) {
	var v uint64
	var shift uint
	for {
		b, err := r.u8()
		if err != nil {
			return 0, err
		}
		v |= uint64(b&0x7f) << shift
		if b&0x80 == 0 {
			return v, nil
		}
		shift += 7
		if shift >= 64 {
			return 0, errors.New("marsdb: varint overflow in batch payload")
		}
	}
}

func (r *batchReader) svarint() (int64, error) {
	v, err := r.varint()
	if err != nil {
		return 0, err
	}
	return int64(v>>1) ^ -int64(v&1), nil
}

func (r *batchReader) bytes(n int) ([]byte, error) {
	if r.pos+n > len(r.buf) {
		return nil, errBatchTruncated
	}
	s := r.buf[r.pos : r.pos+n]
	r.pos += n
	return s, nil
}

func (r *batchReader) lenPrefixedString() (string, error) {
	n, err := r.varint()
	if err != nil {
		return "", err
	}
	b, err := r.bytes(int(n))
	if err != nil {
		return "", err
	}
	return string(b), nil
}

// Graph ids preserve the pre-batch behavior: int64 when it fits,
// uint64 above int64's range.
func idValue(v uint64) any {
	if v > math.MaxInt64 {
		return v
	}
	return int64(v)
}

func (r *batchReader) tableString(table []string) (string, error) {
	id, err := r.varint()
	if err != nil {
		return "", err
	}
	if id >= uint64(len(table)) {
		return "", fmt.Errorf("marsdb: batch string id %d out of range", id)
	}
	return table[id], nil
}

func (r *batchReader) nodeOrEdgeBody(table []string, isNode bool) (map[string]any, error) {
	out := make(map[string]any)
	id, err := r.varint()
	if err != nil {
		return nil, err
	}
	out["id"] = idValue(id)
	if isNode {
		out["__type"] = "node"
		labelCount, err := r.varint()
		if err != nil {
			return nil, err
		}
		labels := make([]any, 0, labelCount)
		for range labelCount {
			label, err := r.tableString(table)
			if err != nil {
				return nil, err
			}
			labels = append(labels, label)
		}
		out["labels"] = labels
	} else {
		out["__type"] = "edge"
		src, err := r.varint()
		if err != nil {
			return nil, err
		}
		dst, err := r.varint()
		if err != nil {
			return nil, err
		}
		out["src"] = idValue(src)
		out["dst"] = idValue(dst)
		label, err := r.tableString(table)
		if err != nil {
			return nil, err
		}
		out["label"] = label
	}
	propCount, err := r.varint()
	if err != nil {
		return nil, err
	}
	props := make(map[string]any, propCount)
	for range propCount {
		name, err := r.tableString(table)
		if err != nil {
			return nil, err
		}
		value, err := r.value(table)
		if err != nil {
			return nil, err
		}
		props[name] = value
	}
	out["props"] = props
	return out, nil
}

func (r *batchReader) value(table []string) (any, error) {
	tag, err := r.u8()
	if err != nil {
		return nil, err
	}
	switch tag {
	case tagNull:
		return nil, nil
	case tagBool:
		b, err := r.u8()
		return b != 0, err
	case tagInt:
		return r.svarint()
	case tagFloat:
		raw, err := r.bytes(8)
		if err != nil {
			return nil, err
		}
		bits := uint64(raw[0]) | uint64(raw[1])<<8 | uint64(raw[2])<<16 | uint64(raw[3])<<24 |
			uint64(raw[4])<<32 | uint64(raw[5])<<40 | uint64(raw[6])<<48 | uint64(raw[7])<<56
		return math.Float64frombits(bits), nil
	case tagString, tagDate, tagDuration:
		return r.lenPrefixedString()
	case tagNode:
		return r.nodeOrEdgeBody(table, true)
	case tagEdge:
		return r.nodeOrEdgeBody(table, false)
	case tagList, tagPath:
		n, err := r.varint()
		if err != nil {
			return nil, err
		}
		items := make([]any, 0, n)
		for range n {
			item, err := r.value(table)
			if err != nil {
				return nil, err
			}
			items = append(items, item)
		}
		return items, nil
	case tagMap:
		n, err := r.varint()
		if err != nil {
			return nil, err
		}
		m := make(map[string]any, n)
		for range n {
			key, err := r.tableString(table)
			if err != nil {
				return nil, err
			}
			value, err := r.value(table)
			if err != nil {
				return nil, err
			}
			m[key] = value
		}
		return m, nil
	default:
		return nil, fmt.Errorf("marsdb: unknown batch value tag 0x%02x", tag)
	}
}

func decodeBatch(buf []byte) ([]map[string]any, Stats, error) {
	r := &batchReader{buf: buf}
	version, err := r.u8()
	if err != nil {
		return nil, Stats{}, err
	}
	if version != 1 {
		return nil, Stats{}, fmt.Errorf("marsdb: unsupported batch version %d", version)
	}
	tableLen, err := r.varint()
	if err != nil {
		return nil, Stats{}, err
	}
	table := make([]string, 0, tableLen)
	for range tableLen {
		s, err := r.lenPrefixedString()
		if err != nil {
			return nil, Stats{}, err
		}
		table = append(table, s)
	}
	columnCount, err := r.varint()
	if err != nil {
		return nil, Stats{}, err
	}
	columns := make([]string, 0, columnCount)
	for range columnCount {
		c, err := r.tableString(table)
		if err != nil {
			return nil, Stats{}, err
		}
		columns = append(columns, c)
	}
	rowCount, err := r.varint()
	if err != nil {
		return nil, Stats{}, err
	}
	rows := make([]map[string]any, 0, rowCount)
	for range rowCount {
		row := make(map[string]any, columnCount)
		for _, col := range columns {
			v, err := r.value(table)
			if err != nil {
				return nil, Stats{}, err
			}
			row[col] = v
		}
		rows = append(rows, row)
	}
	var counters [7]uint64
	for i := range counters {
		counters[i], err = r.varint()
		if err != nil {
			return nil, Stats{}, err
		}
	}
	stats := Stats{
		NodesCreated:         counters[0],
		NodesDeleted:         counters[1],
		RelationshipsCreated: counters[2],
		RelationshipsDeleted: counters[3],
		PropertiesSet:        counters[4],
		LabelsAdded:          counters[5],
		LabelsRemoved:        counters[6],
	}
	return rows, stats, nil
}
