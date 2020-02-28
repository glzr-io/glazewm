using System.Collections.Generic;
using LarsWM.Domain.Common.Models;

namespace LarsWM.Domain.Containers
{
    public static class TreeTraversalHelper
    {
        /// <summary>
        /// Extension method for breadth-first downward traversal from a container list (eg. ContainerForest).
        /// </summary>
        public static IEnumerable<Container> TraverseDownEnumeration(this List<Container> containerList)
        {
            var queue = new Queue<Container>();

            foreach (var tree in containerList)
                queue.Enqueue(tree);

            while (queue.Count > 0)
            {
                var current = queue.Dequeue();
                yield return current;
                foreach (var child in current.Children)
                    queue.Enqueue(child);
            }
        }
        /// <summary>
        /// Extension method for breadth-first downward traversal from a single container.
        /// </summary>
        public static IEnumerable<Container> TraverseDownEnumeration(Container container)
        {
            var queue = new Queue<Container>();
            queue.Enqueue(container);

            while (queue.Count > 0)
            {
                var current = queue.Dequeue();
                yield return current;
                foreach (var child in container.Children)
                    queue.Enqueue(child);
            }
        }

        /// <summary>
        /// Extension method for upwards traversal from a single container.
        /// </summary>
        public static IEnumerable<Container> TraverseUpEnumeration(this Container container)
        {
            var parent = container.Parent;

            while (parent != null)
            {
                yield return parent;
                parent = parent.Parent;
            }
        }
    }
}
