# convex-typegen

A blazing fast Rust type generator for [ConvexDB](https://www.convex.dev) schemas and functions.

## Features

- 🚀 **Blazing Fast**: Efficient AST parsing and type generation using oxc
- 🔄 **Auto-regeneration**: Types automatically update when schema or function files change
- 🛠️ **Complete Type System**: 
  - Full schema type generation (tables, columns, unions)
  - Function argument types for queries, mutations, and actions
  - Support for all Convex types (arrays, objects, records, literals)
  - Proper handling of optional fields and complex types
- 🔒 **Type Safety**: 
  - Compile-time type checking
  - Automatic serialization/deserialization
  - Zero runtime overhead
- 🎨 **Developer Experience**: 
  - Clean, idiomatic Rust code generation
  - Smart function path resolution (e.g., "auth:login")
  - Detailed documentation for generated types

## Quick Start

1. Add dependencies to your `Cargo.toml`:

```toml
convex-typegen = "0.1.0"
```

2. Add the following to your `build.rs` file:

```rust
use convex_typegen::generate;

fn main() {
    generate().unwrap();
}
```

3. Run `cargo build` to generate the types.

You can watch a demo video [here]() to learn more.

## Supported Types

- **Basic Types**: `string`, `number`, `boolean`, `null`, `int64`, `bytes`
- **Complex Types**: `array`, `object`, `record`, `union`, `optional`
- **Special Types**: `any`, `literal`, `id`
- **Custom Types**: Automatic enum generation for union types

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built for use with [ConvexDB](https://convex.dev)
- Powered by [oxc](https://github.com/oxc-project/oxc) for TypeScript parsing

## Related Projects

- [convex rust sdk](https://docs.rs/convex/latest/convex/) - Official Rust client for ConvexDB