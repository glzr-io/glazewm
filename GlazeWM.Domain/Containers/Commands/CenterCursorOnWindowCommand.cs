using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class CenterCursorOnWindowCommand : Command
  {
    public Container TargetContainer { get; }

    /// <summary>
    ///  Center cursor in the middle of target container
    /// </summary>
    public CenterCursorOnWindowCommand(Container target)
    {
      TargetContainer = target;
    }
  }
}
