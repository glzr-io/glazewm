using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class SetNativeFocusCommand : Command
  {
    public Container ContainerToFocus { get; }

    public SetNativeFocusCommand(Container containerToFocus)
    {
      ContainerToFocus = containerToFocus;
    }
  }
}
