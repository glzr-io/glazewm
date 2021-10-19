using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class MoveContainerWithinTreeCommand : Command
  {
    public Container Container { get; }
    public Container Target { get; }
    public InsertionPosition InsertionPosition { get; }

    public MoveContainerWithinTreeCommand(Container container, Container target, InsertionPosition insertionPosition)
    {
      Container = container;
      Target = target;
      InsertionPosition = insertionPosition;
    }
  }
}
