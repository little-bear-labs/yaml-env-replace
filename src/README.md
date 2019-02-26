# env-replace
Replace values in YAML files with environment variables.

### Usage

```yaml
key:
    test:
        - path: ${HOME}
```
Running the given file through env replace using `env-replace -i test.yaml` should result in:
```yaml
key:
    test:
        - path: /home/user
---
```
The result is written to stdout by default but an output file may be specified using `env-replace  -i <input file> -o <output file>`.