using GlazeWM.Infrastructure.Bussing.Commands;

namespace GlazeWM.Infrastructure.Bussing.CommandHandlers
{
  internal class NoopHandler : ICommandHandler<NoopCommand>
  {
    public CommandResponse Handle(NoopCommand command)
    {
      return CommandResponse.Ok;
    }
  }
}
