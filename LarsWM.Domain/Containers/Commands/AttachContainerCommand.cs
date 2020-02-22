using LarsWM.Domain.Common.Models;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Monitors.Commands
{
    public class AttachContainerCommand : Command
    {
        public SplitContainer Parent { get; }
        public Container NewChild { get; }

        public AttachContainerCommand(SplitContainer parent, Container newChild)
        {
            Parent = parent;
            NewChild = newChild;
        }
    }
}
