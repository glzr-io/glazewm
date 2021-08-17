using LarsWM.Domain.Containers.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
{
  class CreateFocusStackHandler : ICommandHandler<CreateFocusStackCommand>
  {
    public dynamic Handle(CreateFocusStackCommand command)
    {
      var target = command.FocusedContainer;

      // Traverse upwards, creating a focus stack from the focused container.
      while (target.Parent != null)
      {
        target.Parent.LastFocusedContainer = target;
        target = target.Parent;
      }

      return CommandResponse.Ok;
    }
  }
}
