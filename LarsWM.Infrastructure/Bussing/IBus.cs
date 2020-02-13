namespace LarsWM.Infrastructure.Bussing
{
    public interface IBus
    {
        CommandResponse Invoke<T>(T command) where T : Command;
        void RaiseEvent<T>(T @event) where T : Event;
        void RegisterCommandHandler<T>();
        void RegisterEventHandler<T>();
    }
}
