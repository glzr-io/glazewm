using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Common.Models;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.Commands
{
    public class ChangeContainerLayout : Command
    {
        public Container Container { get; }
        public Layout Layout { get; }

        public ChangeContainerLayout(Container container, Layout layout)
        {
            Container = container;
            Layout = layout;
        }
    }
}
