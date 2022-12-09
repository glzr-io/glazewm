using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.Commands
{
  public class ActivateBindingModeCommand : Command
  {
    public string BindingMode { get; }

    public ActivateBindingModeCommand(string bindingMode)
    {
      BindingMode = bindingMode;
    }
  }
}
