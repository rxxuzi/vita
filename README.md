# Vita

Vita is a command-line utility designed to recursively traverse directories and output the contents of files.
It intelligently distinguishes between binary and text files (supporting both UTF-8 and Shift-JIS encodings) and provides options to customize its behavior, such as specifying an output file and setting traversal depth.

## Features

- **Recursive Directory Traversal:** Efficiently navigates through directories and their subdirectories.
- **Binary and Text File Detection:** Automatically detects and handles binary and text files, supporting UTF-8 and Shift-JIS encodings.
- **Customizable Output:** Option to write the output to a specified file or standard output.
- **Depth Control:** Ability to limit the depth of directory traversal.
- **Cross-Platform Compatibility:** Supports both MinGW (Windows) and GCC (Linux) environments.

## Installation

### Prerequisites

- **C Compiler:** Ensure you have `gcc` (for Linux) or `MinGW` (for Windows) installed.
- **Make:** Required for building with Makefile.

### Building with Makefile

1. **Clone the Repository:**

   ```bash
   git clone https://github.com/rxxuzi/vita.git
   cd vita
   ```

2. **Compile the Project:**

    - **Linux:**

      ```bash
      make
      ```

    - **Windows (MinGW):**

      ```bash
      mingw32-make
      ```

3. **Executable:**

   After successful compilation, the `vita` executable will be available in the project root directory.

## Tips

### Windows

To copy the project details to the clipboard, adjust the console to use UTF-8 and pipe the output to `clip`:

```bash
chcp 65001
vita sample/ | clip
```

### Linux

To output the project details to a file, use redirection:

```bash
vita sample > out.txt
```

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgements

- **Inspiration:** Inspired by the [uithub](https://github.com/uithub/uithub) project.
- Special thanks to the open-source community for invaluable resources and support.
