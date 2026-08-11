module github.com/knoguchi/marsdb/marsdb-go-arrow

go 1.25.12

require (
	github.com/apache/arrow-go/v18 v18.0.0
	github.com/knoguchi/marsdb/marsdb-go v0.0.0
)

require (
	github.com/goccy/go-json v0.10.3 // indirect
	github.com/google/flatbuffers v24.3.25+incompatible // indirect
	github.com/klauspost/compress v1.17.11 // indirect
	github.com/klauspost/cpuid/v2 v2.2.8 // indirect
	github.com/pierrec/lz4/v4 v4.1.21 // indirect
	github.com/zeebo/xxh3 v1.0.2 // indirect
	golang.org/x/exp v0.0.0-20240909161429-701f63a606c0 // indirect
	golang.org/x/mod v0.21.0 // indirect
	golang.org/x/sync v0.8.0 // indirect
	golang.org/x/sys v0.26.0 // indirect
	golang.org/x/tools v0.26.0 // indirect
	golang.org/x/xerrors v0.0.0-20231012003039-104605ab7028 // indirect
)

// This module links against the locally built marsdb-capi (see README),
// so the sibling checkout is the source of truth for the core binding.
replace github.com/knoguchi/marsdb/marsdb-go => ../marsdb-go
