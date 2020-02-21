using LarsWM.Domain.Common.Models;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Monitors.Commands
{
    public class AttachContainerCommand : Command
    {
        public Container Parent { get; }
        public Container NewChild { get; }

        public AttachContainerCommand(Container parent, Container newChild)
        {
            Parent = parent;
            NewChild = newChild;
        }
    }
}
