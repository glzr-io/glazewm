using System.Collections.Generic;
using LarsWM.Domain.Common.Models;

namespace LarsWM.Domain.Containers
{
    public class ContainerService
    {
        public List<Container> ContainerTree = new List<Container>();
        public List<Container> PendingContainersToRedraw = new List<Container>();
    }
}
