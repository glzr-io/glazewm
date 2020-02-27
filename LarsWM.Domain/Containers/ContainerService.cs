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

        // TODO: Consider renaming to PendingWindowsToRedraw and moving to WindowService.
        public List<Container> PendingContainersToRedraw = new List<Container>();

        /// <summary>
        /// Breadth-first downward traversal from given root container.
        /// </summary>
        public IEnumerable<Container> ContainerTreeEnumeration(Container root)
        {
            var queue = new Queue<Container>();
            queue.Enqueue(root);

            while (queue.Count > 0)
            {
                var current = queue.Dequeue();
                yield return current;
                foreach (var child in root.Children)
                    queue.Enqueue(child);
            }
        }
    }
}
