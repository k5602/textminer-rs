# textminer-rs - PII Redaction Service

A high-performance, privacy-focused microservice for redacting Personally Identifiable Information (PII) from text using BERT-based Named Entity Recognition (NER). Built with Rust and Actix Web for maximum performance and reliability.

## Future Improvements

Planned enhancements for upcoming versions:

- Fine-tuned model for better entity recognition
- Support for additional PII types (emails, phone numbers, etc.)
- Custom entity recognition patterns

## Features

- 🔍 **PII Detection**: Uses BERT-based NER to identify various types of PII including:
  - Person names
  - Locations
  - Organizations

- ⚡ **High Performance**: Built with Rust and Actix-Web for exceptional throughput and low latency
- 🎯 **Simple API**: Easy-to-use HTTP endpoints for integration with any application
- 🔄 **Concurrent Processing**: Handles multiple requests efficiently with async/await
- 🏗️ **Production-Ready**: Includes proper error handling and health checks
- 🚀 **Efficient Model Loading**: Lazy initialization of the BERT model for faster startup

## Getting Started

### Prerequisites

- Rust (latest stable version)
- libtorch (required by rust-bert)

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/k5602/textminer-rs.git
   cd textminer-rs
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

### Running the Service

```bash
cargo run --release
```

The service will start on `http://0.0.0.0:8080`

## API Endpoints

### Single Text Redaction

`POST /api/redact` - Redact PII from a single text

#### Request:

```json
{
  "text": "Your text containing PII like John Smith and New York"
}
```

#### Response:

```json
{
  "redacted_text": "Your text containing PII like [PER] and [LOC]",
  "processing_time_ms": 42,
  "entities_found": 2,
  "entity_types": ["PER", "LOC"],
  "confidence_scores": [0.98, 0.92]
}
```

### Batch Text Redaction

`POST /api/redact/batch` - Redact PII from multiple texts in a single request

#### Request:

```json
{
  "texts": [
    "First text with PII like John Smith",
    "Second text with locations like New York and London"
  ],
  "options": {
    "include_confidence": true
  }
}
```

#### Response:

```json
{
  "results": [
    {
      "redacted_text": "First text with PII like [PER]",
      "processing_time_ms": 0,
      "entities_found": 1,
      "entity_types": ["PER"],
      "confidence_scores": [0.98]
    },
    {
      "redacted_text": "Second text with locations like [LOC] and [LOC]",
      "processing_time_ms": 0,
      "entities_found": 2,
      "entity_types": ["LOC", "LOC"],
      "confidence_scores": [0.95, 0.92]
    }
  ],
  "total_processing_time_ms": 65,
  "total_entities_found": 3
}
```

### Health Check

`GET /api/health` - Health check endpoint

#### Response:

```json
{
  "status": "ok",
  "model_loaded": true
}
```

## Performance

The service is optimized for high throughput and low latency, with the NER model loaded just once at startup and shared across requests using thread-safe reference counting. On average, it processes text in under 100ms, though the first request may take longer as it loads the BERT model into memory.

## Web Interface

Access the web interface at `http://localhost:8080` to test the redaction service interactively.


## Development

### Building for Development

```bash
cargo build## Running with Docker

```bash
# Build the Docker image
docker build -t redactr .

# Run the container
docker run -p 8080:8080 redactr
```
```

### Running Tests

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgements

- [rust-bert](https://github.com/guillaume-be/rust-bert) - Rust native BERT implementation
- [Actix-Web](https://actix.rs/) - Powerful, pragmatic, and extremely fast web framework for Rust


Made with ❤️ in Rust by [Khaled](https://github.com/k5602)