#include <steam_api.h>

class RustSteamCallback final: CCallbackBase {
public:
    RustSteamCallback(
        int parameter_size,
        void *userdata,
        void (*run_func)(void*, void*),
        void (*dealloc)(void *),
        int callback_id
    ): parameter_size(parameter_size), userdata(userdata),
        run_func(run_func), dealloc(dealloc)
    {
        SteamAPI_RegisterCallback(this, callback_id);
    }
    ~RustSteamCallback() {
        SteamAPI_UnregisterCallback(this);
        dealloc(userdata);
    }

	virtual void Run( void *pvParam ) {
        run_func(userdata, pvParam);
    }

	virtual void Run( void *pvParam, bool, SteamAPICall_t) {
        Run( pvParam );
    }

	virtual int GetCallbackSizeBytes() {
        return parameter_size;
    }

private:
    int parameter_size;
    void* userdata;
    void (*run_func)(void *, void *);
    void (*dealloc)(void *);
};


extern "C" void* register_rust_steam_callback(
    int parameter_size,
    void *userdata,
    void (*run_func)(void*, void*),
    void (*dealloc)(void *),
    int callback_id
) {
    return new RustSteamCallback(parameter_size, userdata, run_func, dealloc, callback_id);
}

extern "C" void unregister_rust_steam_callback(
    void* ty
) {
    RustSteamCallback* cb = static_cast<RustSteamCallback*>(ty);
    delete cb;
}