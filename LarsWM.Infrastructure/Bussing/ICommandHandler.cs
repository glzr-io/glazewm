namespace LarsWM.Infrastructure.Bussing
{
    public interface ICommandHandler<TCommand> where TCommand : Command
    {
        void Handle(TCommand command);
    }
}
