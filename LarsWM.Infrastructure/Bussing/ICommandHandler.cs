namespace LarsWM.Infrastructure.Bussing
{
  public interface ICommandHandler<TCommand> where TCommand : Command
  {
    dynamic Handle(TCommand command);
  }
}
