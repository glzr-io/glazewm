using System.Collections.Generic;
using LarsWM.Domain.Common.Models;

namespace LarsWM.Domain.Containers
{
    public class ContainerService
    {
        /// <summary>
        /// List of trees consisting of containers. The root nodes are the monitors,
        /// followed by workspaces, then split containers & windows.
        /// </summary>
        public List<Container> ContainerTree = new List<Container>();

        /// <summary>
        /// Pending SplitContainers to redraw.
        /// </summary>
        public List<SplitContainer> SplitContainersToRedraw = new List<SplitContainer>();
    }
}
