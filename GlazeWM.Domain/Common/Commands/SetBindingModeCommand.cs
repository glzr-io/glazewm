using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.Commands
{
  public class SetBindingModeCommand : Command
  {
    public string BindingModeName { get; }

    public SetBindingModeCommand(string bindingModeName)
    {
      BindingModeName = bindingModeName;
    }
  }
}
