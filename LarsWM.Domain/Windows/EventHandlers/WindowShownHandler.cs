using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi.Events;

namespace LarsWM.Domain.Windows.EventHandlers
{
  class WindowShownHandler : IEventHandler<WindowShownEvent>
  {
    private Bus _bus;

    public WindowShownHandler(Bus bus)
    {
      _bus = bus;
    }

    public void Handle(WindowShownEvent @event)
    {
      _bus.Invoke(new AddWindowCommand(@event.WindowHandle));
    }
  }
}
