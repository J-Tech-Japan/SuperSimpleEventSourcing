# SuperSimpleEventSourcing (Go version)

**A Sample Event Sourcing Demo with Sekiban.Pure**

This repository provides a very simple demonstration of event sourcing.

```
Event sourcing is a simple concept: save events, and project the current state using a projector.
```

Included here are a lightweight event sourcing library and a sample domain, making it a great starting point for understanding how event sourcing frameworks can be built.

It also comes with a minimal Web API (with a scalar UI), a simple console application, and straightforward unit tests.

## Limitations
- Event only saved in memory.
- Event is not generic version. Trait Object Version.

# Project Overview

- [x] **Domain**: A simple event sourcing framework and sample domain. (simple version completed)
- [ ] **Web**: A basic Web API.
- [x] **Console**: A simple console application. (simple version completed)

https://github.com/J-Tech-Japan/SuperSimpleEventSourcing/tree/main/go

- [ ] **Test**: Basic Unit tests. (Not yet)

**Note**: Querying and listing existing aggregates is not currently implemented.

# Key Components

- [x] **Aggregate**: Represents the state container, which changes throughout its lifecycle. (simple version completed)
- [x] **Command**: Triggers changes to an Aggregate. Commands are processed by a command handler and affect only a single partition. (very simple version completed)
- [x] **Command Handler**: A function that produces events in response to a command. (very simple version completed)
- [x] **Event**: A fact that is stored as the source of truth. (simple implementation completed)
- [x] **Partition**: A stream of events. Aggregates can be reconstructed (projected) from these events. (simple implementation completed)
- [x] **Projector**: A function that applies events to evolve the aggregate’s state, returning a new state. (simple implementation completed)

