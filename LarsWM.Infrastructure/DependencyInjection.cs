using LarsWM.Infrastructure.Bussing;
using Microsoft.Extensions.DependencyInjection;

namespace LarsWM.Infrastructure
{
    public static class DependencyInjection
    {
        public static IServiceCollection AddInfrastructureServices(this IServiceCollection services)
        {
            services.AddSingleton<IBus, Bus>();

            // TODO: Change WindowsApiFacade & WindowsApiService to be compatible with DI.

            return services;
        }
    }
}
