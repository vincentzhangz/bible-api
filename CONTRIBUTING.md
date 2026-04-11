# Contributing to Bible API

Thank you for your interest in contributing.

## How to Contribute

### Adding a New Translation

1. **Create a JSON file** in `data/translations/` following the schema at `schema/translation.schema.json`

2. **Validate your JSON**:
   ```bash
   cargo test --test validate_json
   ```

3. **Ensure license compliance**: The `license` field must reference a valid license in `data/licenses/licenses.json`

4. **Submit a pull request** with:
   - The new translation JSON file
   - A brief description of the source

### Reporting Issues

- Report incorrect verses or translation errors
- Report JSON format violations
- Suggest new features

### Code Contributions

1. Fork and create a feature branch
2. Run tests: `cargo test --all-features`
3. Ensure code compiles: `cargo build --all-features`
4. Submit a pull request

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project.
