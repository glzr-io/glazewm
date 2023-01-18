using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class CenterCursorOnContainerCommand : Command
  {
    public Container TargetContainer { get; }

    /// <summary>
    ///  Center cursor in the middle of target container
    /// </summary>
    public CenterCursorOnContainerCommand(Container target)
    {
      TargetContainer = target;
    }
  }
}
