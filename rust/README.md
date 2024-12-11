# SuperSimpleEventSourcing (Rust version)

**A Sample Event Sourcing Demo with Sekiban.Pure**

This repository provides a very simple demonstration of event sourcing.

```
Event sourcing is a simple concept: save events, and project the current state using a projector.
```

Included here are a lightweight event sourcing library and a sample domain, making it a great starting point for understanding how event sourcing frameworks can be built.

It also comes with a minimal Web API (with a scalar UI), a simple console application, and straightforward unit tests.

## Limitations


# Project Overview

- [ ] **Domain**: A simple event sourcing framework and sample domain.
- [ ] **Web**: A basic Web API.
- [ ] **Console**: A simple console application. (Working on NOW)
- [ ] **Test**: Basic Unit tests.

**Note**: Querying and listing existing aggregates is not currently implemented.

# Key Components

- [ ] **Aggregate**: Represents the state container, which changes throughout its lifecycle. (Working on now)
- [ ] **Command**: Triggers changes to an Aggregate. Commands are processed by a command handler and affect only a single partition. (Not yet)
- [ ] **Command Handler**: A function that produces events in response to a command. (Not Yet)
- [x] **Event**: A fact that is stored as the source of truth. (simple implementation completed)
- [x] **Partition**: A stream of events. Aggregates can be reconstructed (projected) from these events. (simple implementation completed)
- [x] **Projector**: A function that applies events to evolve the aggregate’s state, returning a new state. (simple implementation completed)

