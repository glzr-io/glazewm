using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.Commands
{
  public class DeactivateBindingModeCommand : Command
  {
    public string BindingMode { get; }

    public DeactivateBindingModeCommand(string bindingMode)
    {
      BindingMode = bindingMode;
    }
  }
}
