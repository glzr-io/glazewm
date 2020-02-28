using System.Collections.Generic;
using LarsWM.Domain.Common.Models;

namespace LarsWM.Domain.Containers
{
    public static class TreeTraversalHelper
    {
        /// <summary>
        /// Extension method for ContainerForest for iterating downwards.
        /// </summary>
        public static IEnumerable<Container> DownwardsTraversal(this List<Container> containerForest)
        {
            var queue = new Queue<Container>();

            foreach (var tree in containerForest)
                queue.Enqueue(tree);

            while (queue.Count > 0)
            {
                var current = queue.Dequeue();
                yield return current;
                foreach (var child in current.Children)
                    queue.Enqueue(child);
            }
        }

        // TODO: Add traversal method for iterating down from a single container.

        /// <summary>
        /// Extension method for Container for iterating upwards.
        /// </summary>
        public static IEnumerable<Container> UpwardsTraversal(this Container container)
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
