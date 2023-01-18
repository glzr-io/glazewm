using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Commands;

namespace GlazeWM.Infrastructure.Common.CommandHandlers
{
  internal class NoopHandler : ICommandHandler<NoopCommand>
  {
    public CommandResponse Handle(NoopCommand command)
    {
      return CommandResponse.Ok;
    }
  }
}
