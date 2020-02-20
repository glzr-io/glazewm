using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.CommandHandlers
{
    class FocusWindowHandler : ICommandHandler<FocusWindowCommand>
    {
        private IBus _bus;

        public FocusWindowHandler(IBus bus)
        {
            _bus = bus;
        }

        public dynamic Handle(FocusWindowCommand command)
        {
            var window = command.Window;

            // TODO: If already focused, do nothing. If not focused, focus window.

            return new CommandResponse(true, window.Id);
        }
    }
}
