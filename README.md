# tinycache

All the standard stuff that you would expect:

- [X] api to read or fallback to fetch and write item with `get_cached_or_fetch`
- [X] binary serialization
- [X] optional max item age
- [X] tracing logs as a feature
- [X] invalidates if item is not conforming to set type
- [X] uri safe keys with `sha1`
- [X] ~200 lines of rust