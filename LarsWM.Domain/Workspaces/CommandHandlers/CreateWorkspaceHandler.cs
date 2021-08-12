using LarsWM.Infrastructure.Bussing;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces.Commands;

namespace LarsWM.Domain.Workspaces.CommandHandlers
{
  class CreateWorkspaceHandler : ICommandHandler<CreateWorkspaceCommand>
  {
    private WorkspaceService _workspaceService;

    public CreateWorkspaceHandler(WorkspaceService workspaceService)
    {
      _workspaceService = workspaceService;
    }

    public dynamic Handle(CreateWorkspaceCommand command)
    {
      var newWorkspace = new Workspace(command.WorkspaceName);
      _workspaceService.InactiveWorkspaces.Add(newWorkspace);

      return new CommandResponse(true, newWorkspace.Id);
    }
  }
}
