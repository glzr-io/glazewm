using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Workspaces;
using System.Collections.Generic;

namespace LarsWM.Domain.Containers
{
    class ContainerService
    {
        //Tree<Container> Tree = new Tree<Container>();
        public List<Container> ContainerTree = new List<Container>();

        public Monitor GetMonitorForContainer(Container container)
        {
            var parent = container.Parent;

            while (parent != null && parent is Monitor == false)
                parent = container.Parent;

            return parent as Monitor;
        }

        public Workspace GetWorkspaceForContainer(Container container)
        {
            var parent = container.Parent;

            while (parent != null && parent is Workspace == false)
                parent = container.Parent;

            return parent as Workspace;
        }
    }
    // Workspace should have an orientation (horizontal, vertical), but shouldn't extend SplitContainer
    // Doesn't really matter what monitor is to the left or right, it matters what workspace is
    // Should a workspace have the same height/width as the output? Or height/width - gaps?
}
