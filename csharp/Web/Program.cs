using Domain;
using ResultBoxes;
using Scalar.AspNetCore;

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
