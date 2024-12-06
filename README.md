# SuperSimpleEventSourcing
Sample Event Sourcing with Sekiban.Pure

This is super simple event sourcing sample demo.

```
Event Sourcing is simple concept. Save event, and state can be projected with projector.
```

It includes event sourcing library, and sample domain. It's good to study how event sourcing library is made.

Also it have simple WEB API with Scalar UI, Simple Console App and Simple Unit Test.

## Limitation
Event storage is just simple `List<IEvent>`. Which is just runtime storage. Also it only has Partition Level Projection, which can not make live project to list all aggregates.

# Project

- Domain : Simple Event Sourcing Framework and sample Domain
- Web : Simple Web API
- Console : Simple Console App
- Test : Simple xUnit Test


**TODO** : Query and listing existing aggregate is not included.


# Parts

- Aggregate: Box for the State. It can be changed during state cycles.
- Command: Command to make change to Aggregate. It passes into the Command Handler. It can only be make effect on single partition.
- Command Handelr: Function to produce event from command.
- Event: Fact to save as the source of the truth.
- Partition: A Stream to save events. Aggregate can be projected from a stream.
- Projector: A Function to evolve Aggregate with Event. Returns new Aggregate State.

Simple Branch can be defined below.
It has Command to register branch and change Name to the branch.

```SimpleBranchDefinition.cs
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

# Usage
- Register Branch in Console.
```RegisterBranch.cs

Console.WriteLine("input new branch name:");
var inputN = Console.ReadLine();
var responseN = await executor.Execute(new RegisterBranch(inputN)).UnwrapBox();
var aggregateN = Repository.Load<BranchProjector>(responseN.PartitionKeys).UnwrapBox();
Console.WriteLine(JsonSerializer.Serialize(aggregateN.ToTypedPayload<Branch>().UnwrapBox()));

```
- Change Branch Name in Console.

```ChangeBranchName.cs
    Console.WriteLine("ChangeName: input changing name:");
    var input = Console.ReadLine();
    if (!string.IsNullOrEmpty(input))
    {
        var response = await executor.Execute(new ChangeBranchName(responseN.PartitionKeys.AggregateId, input??"")).UnwrapBox();
        var aggregate = Repository.Load<BranchProjector>(response.PartitionKeys).UnwrapBox();
        Console.WriteLine(JsonSerializer.Serialize(aggregate.ToTypedPayload<Branch>().UnwrapBox()));
    }
```

- minimal API definition.
```Program.cs
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
    Repository.Load<BranchProjector>(PartitionKeys<BranchProjector>.Existing(id)).Conveyor(aggregate => aggregate.ToTypedPayload<Branch>()).UnwrapBox()).WithOpenApi();
app.Run();

```

![webAPI usage sample](/output.gif)

# Next?

We are building full event sourcing framework with this concept with Sekiban.

https://github.com/J-Tech-Japan/Sekiban

We are always improving how to write domain. Concept above is still under the development, but Sekiban can use Azure Cosmos DB, Dynamo DB and Postgres. It has full feature to build event sourcing app for Small and Medium application and working to extend distributed environment.
