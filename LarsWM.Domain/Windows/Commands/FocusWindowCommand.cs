using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.Commands
{
    class FocusWindowCommand : Command
    {
        public Window Window { get; set; }

        public FocusWindowCommand(Window window)
        {
            Window = window;
        }
    }
}
