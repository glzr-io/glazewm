using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Common.Models;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.Commands
{
    public class ChangeContainerLayoutCommand : Command
    {
        public Layout NewLayout { get; }

        public ChangeContainerLayoutCommand(Layout newLayout)
        {
            NewLayout = newLayout;
        }
    }
}
