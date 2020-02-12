using System;
using System.Collections.Generic;
using System.Linq;

namespace LarsWM.Domain.Workspaces
{
    class WorkspaceService
    {
        public List<Workspace> Workspaces { get; set; } = new List<Workspace>();

        public Workspace GetWorkspaceById(Guid id)
        {
            return Workspaces.FirstOrDefault(m => m.Id == id);
        }
    }
}
