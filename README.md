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

Below is a simple example using a "Branch" entity. It includes commands to register a branch and to change the branch's name.

```csharp
public record Branch(string Name) : IAggregatePayload;
public record BranchCreated(string Name) : IEventPayload;
public record BranchNameChanged(string Name) : IEventPayload;

public class BranchProjector : IAggregateProjector
{
    public IAggregatePayload Project(IAggregatePayload payload, IEvent ev) =>
        (payload, ev.GetPayload()) switch
        {
            (EmptyAggregatePayload, BranchCreated created) => new Branch(created.Name),
            (Branch branch, BranchNameChanged changed) => new Branch(changed.Name),
            _ => payload
        };
}

public record RegisterBranch(string Name) : ICommandWithHandler<RegisterBranch, BranchProjector>
{
    public PartitionKeys SpecifyPartitionKeys(RegisterBranch command) => PartitionKeys<BranchProjector>.Generate();
    public ResultBox<EventOrNone> Handle(RegisterBranch command, ICommandContext context) =>
        EventOrNone.Event(new BranchCreated(command.Name));
}

public record ChangeBranchName(Guid BranchId, string NameToChange)
    : ICommandWithHandler<ChangeBranchName, BranchProjector>
{
    public ResultBox<EventOrNone> Handle(ChangeBranchName command, ICommandContext context) =>
        context.AppendEvent(new BranchNameChanged(command.NameToChange));
    public PartitionKeys SpecifyPartitionKeys(ChangeBranchName command) =>
        PartitionKeys<BranchProjector>.Existing(BranchId);
}
```

# Usage Examples

**Register a Branch from the Console:**

```csharp
Console.WriteLine("Enter a new branch name:");
var inputN = Console.ReadLine();
var responseN = await executor.Execute(new RegisterBranch(inputN)).UnwrapBox();
var aggregateN = Repository.Load<BranchProjector>(responseN.PartitionKeys).UnwrapBox();
Console.WriteLine(JsonSerializer.Serialize(aggregateN.ToTypedPayload<Branch>().UnwrapBox()));
```

**Change a Branch Name from the Console:**

```csharp
Console.WriteLine("ChangeName: Enter a new name:");
var input = Console.ReadLine();
if (!string.IsNullOrEmpty(input))
{
    var response = await executor.Execute(new ChangeBranchName(responseN.PartitionKeys.AggregateId, input ?? "")).UnwrapBox();
    var aggregate = Repository.Load<BranchProjector>(response.PartitionKeys).UnwrapBox();
    Console.WriteLine(JsonSerializer.Serialize(aggregate.ToTypedPayload<Branch>().UnwrapBox()));
}
```

**Minimal API Definition:**

```csharp
var builder = WebApplication.CreateBuilder(args);
builder.Services.AddOpenApi();
var app = builder.Build();

if (app.Environment.IsDevelopment())
{
    app.MapOpenApi();
    app.MapScalarApiReference();
}
app.MapGet("/", () => "Hello World!");

app.MapPost("/api/branch/register", async (RegisterBranch command) =>
{
    var executor = new CommandExecutor { EventTypes = new DomainEventTypes() };
    return await executor.Execute(command).UnwrapBox();
}).WithOpenApi();

app.MapPost("/api/branch/changename", async (ChangeBranchName command) =>
{
    var executor = new CommandExecutor { EventTypes = new DomainEventTypes() };
    return await executor.Execute(command).UnwrapBox();
}).WithOpenApi();

app.MapGet("/api/branch/{id}", (Guid id) => 
    Repository.Load<BranchProjector>(PartitionKeys<BranchProjector>.Existing(id))
        .Conveyor(aggregate => aggregate.ToTypedPayload<Branch>())
        .UnwrapBox())
    .WithOpenApi();

app.Run();
```

![Web API Usage Sample](/output.gif)

# Next Steps

We are extending these concepts in the Sekiban project:

[https://github.com/J-Tech-Japan/Sekiban](https://github.com/J-Tech-Japan/Sekiban)

We are continually improving domain modeling approaches. While the concepts here are still under development, Sekiban already supports Azure Cosmos DB, DynamoDB, and PostgreSQL. It offers a full-featured environment for building event-sourced applications for small to medium projects, and we are working towards better support in distributed environments.