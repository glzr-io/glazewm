using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class CenterCursorOnContainerCommand : Command
  {
    public Container TargetContainer { get; }

    /// <summary>
    /// Insert child as end element if `targetIndex` is not provided.
    /// </summary>
    public CenterCursorOnContainerCommand(Container target)
    {
      TargetContainer = target;
    }
  }
}
