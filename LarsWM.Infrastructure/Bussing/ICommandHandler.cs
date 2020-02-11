using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Infrastructure.Bussing
{
    public interface ICommandHandler<TCommand> where TCommand : Command
    {
        void Handle(TCommand command);
    }
}
