using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class FlattenSplitContainerCommand : Command
  {
    public SplitContainer ContainerToFlatten { get; }

    public FlattenSplitContainerCommand(SplitContainer parent)
    {
      ContainerToFlatten = parent;
    }
  }
}
