// Firebase stub for MVP
export const auth = {
  currentUser: null, // Mock logged out
  signInWithEmailAndPassword: async () => { /* mock */ },
  onAuthStateChanged: (cb: (user: any) => void) => cb(null), // Mock
};

