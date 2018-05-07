#include <steam_api.h>
#include <steam_gameserver.h>
#include <stdint.h>

struct CallbackData {
    int param_size;
    void* userdata;
    void (*run)(void*, void*, void*);
    void (*run_extra)(void*, void*, void*, uint8_t, SteamAPICall_t);
    void (*dealloc)(void*, void*);
};

class RustCallbackBase final : CCallbackBase {
public:
    RustCallbackBase(uint8 flags, int callback, CallbackData data) : data(data) {
        m_nCallbackFlags = flags;
        m_iCallback = callback;
    }

    ~RustCallbackBase() {
        data.dealloc(this, data.userdata);
    }

    void Run(void* pvParam) {
        data.run(this, data.userdata, pvParam);
    }

    void Run(void* pvParam, bool bIOFailure, SteamAPICall_t hSteamAPICall) {
        data.run_extra(this, data.userdata, pvParam, bIOFailure ? 1 : 0, hSteamAPICall);
    }

    int GetCallbackSizeBytes() {
        return data.param_size;
    }
private:
    CallbackData data;
};

extern "C" {
    void* create_rust_callback(uint8 flags, int id, CallbackData data) {
        return new RustCallbackBase(flags, id, data);
    }

    void delete_rust_callback(void* callback) {
        RustCallbackBase* cb = static_cast<RustCallbackBase*>(callback);
        delete cb;
    }

    int steam_rust_game_server_init(uint32_t ip, uint16_t steam_port, uint16_t game_port,
        uint16_t query_port, EServerMode server_mode, const char* version)
    {
        return SteamGameServer_Init(ip, steam_port, game_port, query_port, server_mode, version);
    }

    int steam_rust_is_steam_id_valid(uint64_t steam_id) {
        CSteamID id = CSteamID();
        id.SetFromUint64(steam_id);
        return id.IsValid();
    }

    int steam_rust_is_game_id_valid(uint64_t game_id) {
        CGameID id = CGameID();
        id.Set(game_id);
        return id.IsValid();
    }

    uint32_t steam_rust_get_game_id_mod(uint64_t game_id) {
        CGameID id = CGameID();
        id.Set(game_id);
        return id.ModID();
    }

    uint32_t steam_rust_get_game_id_app(uint64_t game_id) {
        CGameID id = CGameID();
        id.Set(game_id);
        return id.AppID();
    }

    ISteamClient* steam_rust_get_client() { return SteamClient(); }
    ISteamMatchmaking* steam_rust_get_matchmaking() { return SteamMatchmaking(); }
    ISteamNetworking* steam_rust_get_networking() { return SteamNetworking(); }
    ISteamUtils* steam_rust_get_utils() { return SteamUtils(); }
    ISteamApps* steam_rust_get_apps() { return SteamApps(); }
    ISteamFriends* steam_rust_get_friends() { return SteamFriends(); }
    ISteamUser* steam_rust_get_user() { return SteamUser(); }
    ISteamGameServer* steam_rust_get_server() { return SteamGameServer(); }
    ISteamApps* steam_rust_get_server_apps() { return SteamGameServerApps(); }
}