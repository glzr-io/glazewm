namespace GlazeWM.Infrastructure.Bussing
{
  public interface IEventHandler<TEvent> where TEvent : Event
  {
    void Handle(TEvent @event);
  }
}
