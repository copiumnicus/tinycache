# tinycache

## Features

tinycache is minimal file cache with binary serialization:

- [X] Fallback with `get_cached_or_fetch`
- [X] Binary serialization
- [X] Optional max age for stored values
- [X] optional tracing feature
- [X] Invalidates if fails to read (for fast iteration)
- [X] Uri safe with `sha1`
- [X] 4 deps (serde, bincode, tracing, sha1) 

## Features Overview

Get cached or fetch pattern (read, invalidate if old or non compatible, fetch and write if does not exist):

```rust
let cache_ref = TinyRef::new().max_age(Duration::from_secs(max_age_secs));
let expensive_value = cache_ref.get_cached_or_fetch(item_key, move || fetch_value());
```

Read, write (ya):

```rust
let cache_ref = TinyRef::new();
// write does not return result, it logs tracing if there is an issue
// found that to be more useful
cache_ref.write(key.clone(), &token);
if let Some(v) = cache_ref.read::<Token>(key.clone()) {
    println!("got {:?}", v);
}
```

Under the hood using bincode for small file fast read/write:

```rust
let bytes = bincode::serialize(value).map_err(StoreErr::ser)?;
```

Item keys formatted using sha1:

```rust
fn fmt_key(k: String) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(k.as_bytes());
    hasher.finalize().to_vec()
}
```

Tracing dep and logging can be disabled with a feature:
```toml
[features]
default = ["tracing"]
tracing = ["dep:tracing"]
```