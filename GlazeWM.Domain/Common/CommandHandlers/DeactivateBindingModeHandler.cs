using GlazeWM.Domain.Common.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.CommandHandlers
{
  public class DeactivateBindingModeHandler : ICommandHandler<DeactivateBindingModeCommand>
  {
    public CommandResponse Handle(DeactivateBindingModeCommand command)
    {
      var bindingMode = command.BindingMode;

      return CommandResponse.Ok;
    }
  }
}
