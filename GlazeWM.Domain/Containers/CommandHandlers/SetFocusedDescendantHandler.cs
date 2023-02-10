using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Utils;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class SetFocusedDescendantHandler : ICommandHandler<SetFocusedDescendantCommand>
  {
    public CommandResponse Handle(SetFocusedDescendantCommand command)
    {
      var focusedDescendant = command.FocusedDescendant;
      var endAncestor = command.EndAncestor;

      // Traverse upwards, setting the container as the last focused until the root container
      // or `endAncestor` (if provided) is reached.
      var target = focusedDescendant;
      while (target.Parent != null && target != endAncestor)
      {
        target.Parent.ChildFocusOrder.MoveToFront(target);
        target = target.Parent;
      }

      return CommandResponse.Ok;
    }
  }
}
