"use client";

import { useState } from "react";

const API_BASE = "http://localhost:3001";

type Genre = {
    id: string;
    name: string;
    listeners: number;
};

type UserProfile = {
    id: string;
    username: string;
    email: string;
    subscription_level: number;
};

type Wallet = {
    coin_balance: number;
    credit_balance: number;
};

export default function Home() {
    const [activeTab, setActiveTab] = useState("genres");
    const [genres, setGenres] = useState<Genre[]>([]);
    const [profile, setProfile] = useState<UserProfile | null>(null);
    const [wallet, setWallet] = useState<Wallet | null>(null);
    const [message, setMessage] = useState("");

    // Genre form
    const [genreId, setGenreId] = useState("");
    const [genreName, setGenreName] = useState("");
    const [genreListeners, setGenreListeners] = useState("");

    // User form
    const [userId, setUserId] = useState("user:1234");
    const [username, setUsername] = useState("");
    const [email, setEmail] = useState("");
    const [subLevel, setSubLevel] = useState("0");

    // Transfer form
    const [transferAmount, setTransferAmount] = useState("");

    const showMessage = (msg: string) => {
        setMessage(msg);
        setTimeout(() => setMessage(""), 3000);
    };

    const fetchGenres = async () => {
        try {
            const res = await fetch(`${API_BASE}/genres`);
            const data = await res.json();
            setGenres(data);
            showMessage("Genres loaded!");
        } catch (err) {
            showMessage("Error loading genres");
        }
    };

    const createGenre = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            await fetch(`${API_BASE}/genres`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    id: genreId,
                    name: genreName,
                    listeners: parseInt(genreListeners),
                }),
            });
            showMessage("Genre created!");
            setGenreId("");
            setGenreName("");
            setGenreListeners("");
            fetchGenres();
        } catch (err) {
            showMessage("Error creating genre");
        }
    };

    const fetchProfile = async () => {
        try {
            const res = await fetch(`${API_BASE}/users/${userId}`);
            const data = await res.json();
            setProfile(data);
            showMessage("Profile loaded!");
        } catch (err) {
            showMessage("Error loading profile");
        }
    };

    const createUser = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            await fetch(`${API_BASE}/users`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    id: userId,
                    username,
                    email,
                    subscription_level: parseInt(subLevel),
                }),
            });
            showMessage("User created!");
            fetchProfile();
        } catch (err) {
            showMessage("Error creating user");
        }
    };

    const updateUser = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            await fetch(`${API_BASE}/users/${userId}`, {
                method: "PUT",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    id: userId,
                    username,
                    email,
                    subscription_level: parseInt(subLevel),
                }),
            });
            showMessage("User updated!");
            fetchProfile();
        } catch (err) {
            showMessage("Error updating user");
        }
    };

    const fetchWallet = async () => {
        try {
            const res = await fetch(`${API_BASE}/users/${userId}/wallet`);
            const data = await res.json();
            setWallet(data);
            showMessage("Wallet loaded!");
        } catch (err) {
            showMessage("Error loading wallet");
        }
    };

    const transferCredits = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            await fetch(`${API_BASE}/users/${userId}/wallet/transfer`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ amount: parseInt(transferAmount) }),
            });
            showMessage("Transfer complete!");
            setTransferAmount("");
            fetchWallet();
        } catch (err) {
            showMessage("Error transferring credits");
        }
    };

    const tabs = [
        { id: "genres", label: "Genres" },
        { id: "users", label: "Users" },
        { id: "wallet", label: "Wallet" },
    ];

    return (
        <div className="min-h-screen bg-zinc-50 dark:bg-zinc-950 p-4">
            <div className="mx-auto max-w-2xl">
                <h1 className="mb-6 text-3xl font-bold text-zinc-900 dark:text-zinc-50">
                    Redis API Client
                </h1>

                {/* Message Toast */}
                {message && (
                    <div className="mb-4 rounded-lg bg-green-100 p-3 text-green-800 dark:bg-green-900 dark:text-green-200">
                        {message}
                    </div>
                )}

                {/* Tabs */}
                <div className="mb-6 flex gap-2 overflow-x-auto">
                    {tabs.map((tab) => (
                        <button
                            key={tab.id}
                            onClick={() => setActiveTab(tab.id)}
                            className={`rounded-lg px-4 py-2 font-medium transition-colors ${
                                activeTab === tab.id
                                    ? "bg-zinc-900 text-white dark:bg-zinc-50 dark:text-zinc-900"
                                    : "bg-white text-zinc-700 hover:bg-zinc-100 dark:bg-zinc-800 dark:text-zinc-300 dark:hover:bg-zinc-700"
                            }`}
                        >
                            {tab.label}
                        </button>
                    ))}
                </div>

                {/* Genres Tab */}
                {activeTab === "genres" && (
                    <div className="space-y-6">
                        <div className="rounded-lg bg-white p-6 shadow dark:bg-zinc-900">
                            <h2 className="mb-4 text-xl font-semibold text-zinc-900 dark:text-zinc-50">
                                Create Genre
                            </h2>
                            <form onSubmit={createGenre} className="space-y-4">
                                <input
                                    type="text"
                                    placeholder="Genre ID (e.g., ROCK)"
                                    value={genreId}
                                    onChange={(e) => setGenreId(e.target.value)}
                                    className="w-full rounded-lg border border-zinc-300 p-3 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-50"
                                    required
                                />
                                <input
                                    type="text"
                                    placeholder="Genre Name"
                                    value={genreName}
                                    onChange={(e) => setGenreName(e.target.value)}
                                    className="w-full rounded-lg border border-zinc-300 p-3 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-50"
                                    required
                                />
                                <input
                                    type="number"
                                    placeholder="Listeners"
                                    value={genreListeners}
                                    onChange={(e) => setGenreListeners(e.target.value)}
                                    className="w-full rounded-lg border border-zinc-300 p-3 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-50"
                                    required
                                />
                                <button
                                    type="submit"
                                    className="w-full rounded-lg bg-zinc-900 py-3 font-medium text-white hover:bg-zinc-800 dark:bg-zinc-50 dark:text-zinc-900 dark:hover:bg-zinc-200"
                                >
                                    Create Genre
                                </button>
                            </form>
                        </div>

                        <div className="rounded-lg bg-white p-6 shadow dark:bg-zinc-900">
                            <h2 className="mb-4 text-xl font-semibold text-zinc-900 dark:text-zinc-50">
                                All Genres
                            </h2>
                            <button
                                onClick={fetchGenres}
                                className="mb-4 w-full rounded-lg bg-zinc-900 py-3 font-medium text-white hover:bg-zinc-800 dark:bg-zinc-50 dark:text-zinc-900 dark:hover:bg-zinc-200"
                            >
                                Load Genres
                            </button>
                            <div className="space-y-2">
                                {genres.map((genre) => (
                                    <div
                                        key={genre.id}
                                        className="rounded-lg border border-zinc-200 p-4 dark:border-zinc-700"
                                    >
                                        <div className="font-semibold text-zinc-900 dark:text-zinc-50">
                                            {genre.name}
                                        </div>
                                        <div className="text-sm text-zinc-600 dark:text-zinc-400">
                                            ID: {genre.id} • {genre.listeners.toLocaleString()}{" "}
                                            listeners
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>
                )}

                {/* Users Tab */}
                {activeTab === "users" && (
                    <div className="space-y-6">
                        <div className="rounded-lg bg-white p-6 shadow dark:bg-zinc-900">
                            <h2 className="mb-4 text-xl font-semibold text-zinc-900 dark:text-zinc-50">
                                User ID
                            </h2>
                            <input
                                type="text"
                                placeholder="User ID"
                                value={userId}
                                onChange={(e) => setUserId(e.target.value)}
                                className="w-full rounded-lg border border-zinc-300 p-3 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-50"
                            />
                        </div>

                        <div className="rounded-lg bg-white p-6 shadow dark:bg-zinc-900">
                            <h2 className="mb-4 text-xl font-semibold text-zinc-900 dark:text-zinc-50">
                                Get Profile
                            </h2>
                            <button
                                onClick={fetchProfile}
                                className="mb-4 w-full rounded-lg bg-zinc-900 py-3 font-medium text-white hover:bg-zinc-800 dark:bg-zinc-50 dark:text-zinc-900 dark:hover:bg-zinc-200"
                            >
                                Load Profile
                            </button>
                            {profile && (
                                <div className="rounded-lg border border-zinc-200 p-4 dark:border-zinc-700">
                                    <div className="font-semibold text-zinc-900 dark:text-zinc-50">
                                        {profile.username}
                                    </div>
                                    <div className="text-sm text-zinc-600 dark:text-zinc-400">
                                        {profile.email}
                                    </div>
                                    <div className="text-sm text-zinc-600 dark:text-zinc-400">
                                        Level: {profile.subscription_level}
                                    </div>
                                </div>
                            )}
                        </div>

                        <div className="rounded-lg bg-white p-6 shadow dark:bg-zinc-900">
                            <h2 className="mb-4 text-xl font-semibold text-zinc-900 dark:text-zinc-50">
                                Create/Update User
                            </h2>
                            <form onSubmit={createUser} className="space-y-4">
                                <input
                                    type="text"
                                    placeholder="Username"
                                    value={username}
                                    onChange={(e) => setUsername(e.target.value)}
                                    className="w-full rounded-lg border border-zinc-300 p-3 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-50"
                                    required
                                />
                                <input
                                    type="email"
                                    placeholder="Email"
                                    value={email}
                                    onChange={(e) => setEmail(e.target.value)}
                                    className="w-full rounded-lg border border-zinc-300 p-3 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-50"
                                    required
                                />
                                <select
                                    value={subLevel}
                                    onChange={(e) => setSubLevel(e.target.value)}
                                    className="w-full rounded-lg border border-zinc-300 p-3 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-50"
                                >
                                    <option value="0">Free</option>
                                    <option value="1">Basic</option>
                                    <option value="2">Premium</option>
                                </select>
                                <div className="flex gap-2">
                                    <button
                                        type="submit"
                                        className="flex-1 rounded-lg bg-zinc-900 py-3 font-medium text-white hover:bg-zinc-800 dark:bg-zinc-50 dark:text-zinc-900 dark:hover:bg-zinc-200"
                                    >
                                        Create
                                    </button>
                                    <button
                                        type="button"
                                        onClick={updateUser}
                                        className="flex-1 rounded-lg border border-zinc-900 py-3 font-medium text-zinc-900 hover:bg-zinc-100 dark:border-zinc-50 dark:text-zinc-50 dark:hover:bg-zinc-800"
                                    >
                                        Update
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                )}

                {/* Wallet Tab */}
                {activeTab === "wallet" && (
                    <div className="space-y-6">
                        <div className="rounded-lg bg-white p-6 shadow dark:bg-zinc-900">
                            <h2 className="mb-4 text-xl font-semibold text-zinc-900 dark:text-zinc-50">
                                User ID
                            </h2>
                            <input
                                type="text"
                                placeholder="User ID"
                                value={userId}
                                onChange={(e) => setUserId(e.target.value)}
                                className="w-full rounded-lg border border-zinc-300 p-3 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-50"
                            />
                        </div>

                        <div className="rounded-lg bg-white p-6 shadow dark:bg-zinc-900">
                            <h2 className="mb-4 text-xl font-semibold text-zinc-900 dark:text-zinc-50">
                                Wallet Balance
                            </h2>
                            <button
                                onClick={fetchWallet}
                                className="mb-4 w-full rounded-lg bg-zinc-900 py-3 font-medium text-white hover:bg-zinc-800 dark:bg-zinc-50 dark:text-zinc-900 dark:hover:bg-zinc-200"
                            >
                                Load Wallet
                            </button>
                            {wallet && (
                                <div className="grid grid-cols-2 gap-4">
                                    <div className="rounded-lg border border-zinc-200 p-4 dark:border-zinc-700">
                                        <div className="text-sm text-zinc-600 dark:text-zinc-400">
                                            Coins
                                        </div>
                                        <div className="text-2xl font-bold text-zinc-900 dark:text-zinc-50">
                                            {wallet.coin_balance}
                                        </div>
                                    </div>
                                    <div className="rounded-lg border border-zinc-200 p-4 dark:border-zinc-700">
                                        <div className="text-sm text-zinc-600 dark:text-zinc-400">
                                            Credits
                                        </div>
                                        <div className="text-2xl font-bold text-zinc-900 dark:text-zinc-50">
                                            {wallet.credit_balance}
                                        </div>
                                    </div>
                                </div>
                            )}
                        </div>

                        <div className="rounded-lg bg-white p-6 shadow dark:bg-zinc-900">
                            <h2 className="mb-4 text-xl font-semibold text-zinc-900 dark:text-zinc-50">
                                Transfer Credits
                            </h2>
                            <form onSubmit={transferCredits} className="space-y-4">
                                <input
                                    type="number"
                                    placeholder="Amount (coins → credits)"
                                    value={transferAmount}
                                    onChange={(e) => setTransferAmount(e.target.value)}
                                    className="w-full rounded-lg border border-zinc-300 p-3 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-50"
                                    required
                                />
                                <button
                                    type="submit"
                                    className="w-full rounded-lg bg-zinc-900 py-3 font-medium text-white hover:bg-zinc-800 dark:bg-zinc-50 dark:text-zinc-900 dark:hover:bg-zinc-200"
                                >
                                    Transfer
                                </button>
                            </form>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}