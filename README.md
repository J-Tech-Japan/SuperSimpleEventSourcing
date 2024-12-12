# SuperSimpleEventSourcing

**A Sample Event Sourcing Demo with Sekiban.Pure**

This repository provides a very simple demonstration of event sourcing.

```
Event sourcing is a simple concept: save events, and project the current state using a projector.
```

Included here are a lightweight event sourcing library and a sample domain, making it a great starting point for understanding how event sourcing frameworks can be built.

It also comes with a minimal Web API (with a scalar UI), a simple console application, and straightforward unit tests.

## Limitations

Events are stored in a simple `List<IEvent>`—this is purely in-memory and not meant as a production-ready persistence layer. Additionally, we only have projection at the partition level, so you cannot create live projections that list all aggregates.

# Project Overview

- **Domain**: A simple event sourcing framework and sample domain.
- **Web**: A basic Web API.
- **Console**: A simple console application.
- **Test**: Basic xUnit tests.

**Note**: Querying and listing existing aggregates is not currently implemented.

# Key Components

- **Aggregate**: Represents the state container, which changes throughout its lifecycle.
- **Command**: Triggers changes to an Aggregate. Commands are processed by a command handler and affect only a single partition.
- **Command Handler**: A function that produces events in response to a command.
- **Event**: A fact that is stored as the source of truth.
- **Partition**: A stream of events. Aggregates can be reconstructed (projected) from these events.
- **Projector**: A function that applies events to evolve the aggregate’s state, returning a new state.

# Language 1. (C#)

C# is the language we first developed Sekiban. This is updated concept of new Sekiban. It is still very simple yet powerful enough to extend in future Sekiban.

https://github.com/J-Tech-Japan/SuperSimpleEventSourcing/tree/main/csharp

![Web API Usage Sample](/output.gif)

# Language 2. (Rust)
With my limited knowledge and help of LLM, I have completed very very simple Rust implementation. In comsole you can see it can send command and change aggregate.

https://github.com/J-Tech-Japan/SuperSimpleEventSourcing/tree/main/rust

# Other Languages might coming...

- Go ?
- Typescript ?
- F# 

maybe.

# Next Steps

We are extending these concepts in the Sekiban project:

[https://github.com/J-Tech-Japan/Sekiban](https://github.com/J-Tech-Japan/Sekiban)

We are continually improving domain modeling approaches. While the concepts here are still under development, Sekiban already supports Azure Cosmos DB, DynamoDB, and PostgreSQL. It offers a full-featured environment for building event-sourced applications for small to medium projects, and we are working towards better support in distributed environments.