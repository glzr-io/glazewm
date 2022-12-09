using GlazeWM.Domain.Common.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.CommandHandlers
{
  public class ActivateBindingModeHandler : ICommandHandler<ActivateBindingModeCommand>
  {
    public CommandResponse Handle(ActivateBindingModeCommand command)
    {
      var bindingMode = command.BindingMode;

      return CommandResponse.Ok;
    }
  }
}
