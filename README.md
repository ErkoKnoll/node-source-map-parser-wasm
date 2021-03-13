Experimental source map parser built in Rust using [sourcemap](https://docs.rs/sourcemap/6.0.1/sourcemap/) crate and compiled to WASM.

This package is built for benchmarking purpose and is not actively supported, you should be using [source-map](https://www.npmjs.com/package/source-map) which is also significantly faster. If you want predictable performance you can also look into [node-source-map-parser-native](https://github.com/ErkoKnoll/node-source-map-parser-native).

# Usage (if you really need to use it):
```
import { parse_source_map, lookup_original_position, dispose } from 'node-source-map-parser-wasm';

const file = readFileSync("bundled.js.map");

// Parse source map file and acquire handle for later use
const handle = parse_source_map(file);

// Look up original position
const originalPosition = lookup_original_position(handle, 1, 2);

// Print out original position line, column and source
console.log(originalPosition.line, originalPosition.column, originalPosition.source);

// Dispose parsed source map resources, if not done then you will leak memory
dispose(handle);
```