using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Workspaces.Commands;

namespace GlazeWM.Domain.Workspaces.CommandHandlers
{
  class CreateWorkspaceHandler : ICommandHandler<CreateWorkspaceCommand>
  {
    private WorkspaceService _workspaceService;

    public CreateWorkspaceHandler(WorkspaceService workspaceService)
    {
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(CreateWorkspaceCommand command)
    {
      var newWorkspace = new Workspace(command.WorkspaceName);
      _workspaceService.InactiveWorkspaces.Add(newWorkspace);

      return CommandResponse.Ok;
    }
  }
}
