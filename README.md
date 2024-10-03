# SlickHash: Sliding Block Hashing Implementation

This repository contains the Rust implementation of **Slick Hash**, a lightweight hash table based on the ideas presented by Lehmann, Sanders, and Walzer in their paper, [_Sliding Block Hashing (Slick) -- Basic Algorithmic Ideas_](https://arxiv.org/abs/2304.09283). Slick Hash aims to provide an efficient balance between space consumption and speed, making it an appealing alternative to more traditional hash tables.

## Overview

The Slick Hash algorithm improves hash table efficiency by dividing the table into blocks and using dynamic sliding techniques to manage entries within these blocks. This method helps to keep the table space-efficient while maintaining fast lookup and insertion times.

**Key Features**
- **Main Table and Backyard Management**: Slick Hash uses a primary table for standard storage and a secondary "backyard" to handle overflow entries based on threshold values.
- **Sliding Blocks**: When a block in the main table becomes too full, Slick Hash slides adjacent blocks to create more space, reducing the likelihood of collisions and improving memory efficiency.
- **Backyard Cleaning**: A unique mechanism for deleting and reintegrating entries from the backyard into the main table, though currently an area for further optimization.

## Results and Performance

The implementation has been benchmarked against Rust's built-in `HashMap` and `BinaryTreeMap` in terms of:
- Execution time
- Cache misses
- Branch misses

### Key Findings:
- **Memory Efficiency**: Slick Hash consistently shows better memory efficiency compared to BinaryTreeMap, and competes well with HashMap when querying.
- **Execution Time**: While insertion time increases with higher block sizes, Slick Hash remains competitive with standard implementations for querying, especially when memory footprint is a priority.
- **Hyperparameter Impact**: Fine-tuning the block size and sliding parameters reveals that the default configuration proposed by the authors strikes an excellent balance between space and speed. For detailed analysis, including plots and additional configurations, refer to the report.

### Future Work:
- **Backyard Cleaning Optimization**: Currently, the method used for reintegrating backyard entries into the main table has performance limitations. Further improvements could include parallelization or more sophisticated entry management techniques.
- **Load Factor Handling**: At higher load factors, insertion and query times slightly increase. Optimizations here could further improve the overall performance of Slick Hash under heavy usage.

## Report

For a detailed analysis of the Slick Hash algorithm, please refer to the report available on [arXiv](https://arxiv.org/abs/2409.20125).
