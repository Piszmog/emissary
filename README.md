# Emissary

Emissary is a proxy service written in Rust. It can be used to log boundary events of a service, such as the start and
end of an HTTP request.

## Usage

```shell
$ ./emissary -c emissary.toml
```

### Options

| Option                | Default           | Required  | Description                        |
|-----------------------|:------------------|-----------|------------------------------------|
| `-c`, `--config-file` | `./emissary.toml` | **True**  | The path to the configuration file |
| `-h`, `--help`        | N/A               | **False** | Print help information             |
| `-V`, `--version`     | N/A               | **False** | Print version information          |

## Configuration Schema

Below is the schema for the configuration file.

### `http`

The HTTP `address` and `port` to run **Emissary** on.

###### Example

```toml
[http]
address = "127.0.0.1"
port = 8080
```

### `proxy`

The HTTP `address` and `port` the main application is on.

###### Example

```toml
[proxy]
address = "127.0.0.1"
port = 8082
```

### `logging`

Configures the logging of **Emissary**.

#### `mode`

Determines the logging mode of **Emissary**. There are two options: `json` and `plain`.

###### Example

```toml
[logging]
# Logs are written in JSON format
mode = "json"
```

```toml
[logging]
# Logs are written in plain format
mode = "plain"
```

#### `json.format`

Configures the format of the JSON logs.

###### Example

```toml
[logging.json.format]
request.id = "%{UUID()}"
request.method = "%{METHOD}"
request.uri = "%{URI}"
```

```json
{
  "request": {
    "id": "b5f8f8f8-f8f8-f8f8-f8f8-f8f8f8f8f8f8",
    "method": "GET",
    "uri": "/"
  }
}
```

#### `plain.format`

Configures the format of the plain logs.

###### Example

```toml
[logging.plain.format]
ID = "%{UUID()}"
Method = "%{METHOD}"
URI = "%{URI}"
```

```text
2022-06-10T01:50:23.490857Z  INFO ID="c9035e94-fc3c-43a3-a4ca-93aecbb7b49d" Method="GET" Status="200" URI="/get"
```

## Logging Operations

Logging as a number of builtin operations that can be used.

| Operation             | Description                                                                                        |
|-----------------------|----------------------------------------------------------------------------------------------------|
| `UUID()`              | Generates a UUID                                                                                   |
| `DURATION(NS/MS/S)`   | Returns the duration of the request to the service in either nanoseconds, milliseconds, or seconds |
| `REQUEST_HEADER(..)`  | Returns the specified request header                                                               |
| `RESPONSE_HEADER(..)` | Returns the specified response header                                                              |
| `METHOD`              | Returns the request method                                                                         |
| `URI`                 | Returns the request URI                                                                            |
| `STATUS_CODE`         | Returns the response status code                                                                   |

## Example

See the [emissary.toml](examples/mainapp/emissary.toml) in the `examples` directory.

### JSON

```json
{
  "details": {
    "duration_ms": "1",
    "duration_ns": "1165694",
    "duration_s": "0"
  },
  "request": {
    "id": "8c51cff7-0a8b-48c2-9bc1-2712c3712f79",
    "method": "GET",
    "uri": "/get",
    "version": "1.0.0"
  },
  "response": {
    "content_type": "text/plain; charset=utf-8",
    "status_code": 200
  }
}
```

### Plain

```text
2022-06-10T01:50:23.490857Z  INFO ContentType="text/plain; charset=utf-8" Duration(ms)="1" Duration(ns)="1314047" Duration(s)="0" ID="c9035e94-fc3c-43a3-a4ca-93aecbb7b49d" Method="GET" Status="200" URI="/get" Version="1.0.0" 
```

## TODOs

* Database Logging
* Message Queue Logging
* Add support for custom extensions
* Log output