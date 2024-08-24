# Context

# Example


``` bash
export DATABASE_URL="postgresql://admin:admin@localhost:5432/test"
sea-orm-cli migrate up
```

# 生成 entity 從`test`資料庫底到`src/entity` 範例會生成`wallet_source.rs`
```
sea-orm-cli generate entity \
    -u "postgresql://admin:admin@localhost:5432/test" \
    -o src/entity
```

Example
├── Cargo.toml
├── migration
│   └── ...
└── src
    ├── entity
    │   └── wallet_source.rs
    └── main.rs

