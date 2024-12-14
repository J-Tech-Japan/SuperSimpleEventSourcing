using ResultBoxes;
namespace Domain;

#region User
public record UnconfirmedUser(string Name, string Email) : IAggregatePayload;
public record ConfirmedUser(string Name, string Email) : IAggregatePayload;
public record UserRegistered(string Name, string Email) : IEventPayload;
public record UserConfirmed : IEventPayload;
public record UserUnconfirmed : IEventPayload;
public class UserProjector : IAggregateProjector
{
    public IAggregatePayload Project(IAggregatePayload payload, IEvent ev) => (payload, ev.GetPayload()) switch
    {
        (EmptyAggregatePayload, UserRegistered registered) => new UnconfirmedUser(registered.Name, registered.Email),
        (UnconfirmedUser unconfirmedUser, UserConfirmed) => new ConfirmedUser(
            unconfirmedUser.Name,
            unconfirmedUser.Email),
        (ConfirmedUser confirmedUser, UserUnconfirmed) => new UnconfirmedUser(confirmedUser.Name, confirmedUser.Email),
        _ => payload
    };
    public string GetVersion() => "1.0.1";
    public static Func<IAggregatePayload, IEvent, IAggregatePayload> Projector() =>
        (payload, ev) => (payload, ev.GetPayload()) switch
        {
            (EmptyAggregatePayload, UserRegistered registered) => new UnconfirmedUser(
                registered.Name,
                registered.Email),
            (UnconfirmedUser unconfirmedUser, UserConfirmed) => new ConfirmedUser(
                unconfirmedUser.Name,
                unconfirmedUser.Email),
            (ConfirmedUser confirmedUser, UserUnconfirmed) => new UnconfirmedUser(
                confirmedUser.Name,
                confirmedUser.Email),
            _ => payload
        };
}
public record RegisterUser(string Name, string Email)
    : ICommandWithHandlerInjection<RegisterUser, UserProjector, RegisterUser.Injection>
{
    public PartitionKeys SpecifyPartitionKeys(RegisterUser command) => PartitionKeys<UserProjector>.Generate();
    public ResultBox<EventOrNone> Handle(RegisterUser command, Injection injection, ICommandContext context) =>
        ResultBox
            .Start
            .Conveyor(m => injection.EmailExists(command.Email).ToResultBox())
            .Verify(
                exists => exists
                    ? ExceptionOrNone.FromException(new ApplicationException("Email already exists"))
                    : ExceptionOrNone.None)
            .Conveyor(_ => EventOrNone.Event(new UserRegistered(command.Name, command.Email)));
    public record Injection(Func<string, bool> EmailExists);
}
#endregion

#region Branch
public record Branch(string Name,string Country) : IAggregatePayload;
public record BranchCreated(string Name, string Country) : IEventPayload;
public record BranchNameChanged(string Name) : IEventPayload;
public class BranchProjector : IAggregateProjector
{
    public IAggregatePayload Project(IAggregatePayload payload, IEvent ev) =>
        (payload, ev.GetPayload()) switch
        {
            (EmptyAggregatePayload, BranchCreated created) => new Branch(created.Name, created.Country),
            (Branch branch, BranchNameChanged changed) => branch with {Name = changed.Name},
            _ => payload
        };
}
public record RegisterBranch(string Name, string Country) : ICommandWithHandler<RegisterBranch, BranchProjector>
{
    public PartitionKeys SpecifyPartitionKeys(RegisterBranch command) => PartitionKeys<BranchProjector>.Generate();
    public ResultBox<EventOrNone> Handle(RegisterBranch command, ICommandContext context) =>
        EventOrNone.Event(new BranchCreated(command.Name, command.Country));
}
public record ChangeBranchName(Guid BranchId, string NameToChange)
    : ICommandWithHandler<ChangeBranchName, BranchProjector>
{
    public ResultBox<EventOrNone> Handle(ChangeBranchName command, ICommandContext context) =>
        context.AppendEvent(new BranchNameChanged(command.NameToChange));
    public PartitionKeys SpecifyPartitionKeys(ChangeBranchName command) =>
        PartitionKeys<BranchProjector>.Existing(BranchId);
}
public record RegisterCommand2(string Name, Guid BranchId, string TenantCode) : ICommand;
public class RegisterCommand2Handler : ICommandHandler<RegisterCommand2>, ICommandPartitionSpecifier<RegisterCommand2>
{
    public ResultBox<EventOrNone> Handle(RegisterCommand2 command, ICommandContext context) =>
        throw new NotImplementedException();
    public PartitionKeys SpecifyPartitionKeys(RegisterCommand2 command) => throw new NotImplementedException();
}
public record RegisterCommand3(string Name, string Country) : ICommand, ICommandHandler<RegisterCommand3>
{
    public ResultBox<EventOrNone> Handle(RegisterCommand3 command, ICommandContext context) =>
        EventOrNone.Event(new BranchCreated(command.Name, command.Country));
}
#endregion

#region EventTypes (This should be generated but not have source generator yet. So I wrote it manually.)
public class DomainEventTypes : IEventTypes
{
    public ResultBox<IEvent> GenerateTypedEvent(
        IEventPayload payload,
        PartitionKeys partitionKeys,
        string sortableUniqueId,
        long version) => payload switch
    {
        UserRegistered userRegistered => new Event<UserRegistered>(
            userRegistered,
            partitionKeys,
            sortableUniqueId,
            version),
        UserConfirmed userConfirmed => new Event<UserConfirmed>(
            userConfirmed,
            partitionKeys,
            sortableUniqueId,
            version),
        UserUnconfirmed userUnconfirmed => new Event<UserUnconfirmed>(
            userUnconfirmed,
            partitionKeys,
            sortableUniqueId,
            version),
        BranchCreated branchCreated => new Event<BranchCreated>(
            branchCreated,
            partitionKeys,
            sortableUniqueId,
            version),
        BranchNameChanged branchNameChanged => new Event<BranchNameChanged>(
            branchNameChanged,
            partitionKeys,
            sortableUniqueId,
            version),
        _ => ResultBox<IEvent>.FromException(
            new SekibanEventTypeNotFoundException($"Event Type {payload.GetType().Name} Not Found"))
    };
}
#endregion