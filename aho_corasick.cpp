// Basic implementation of the Aho Corasick algorithm

#include <iostream>
#include <queue>
#include <vector>
#include <list>
#include <string>

// letters a-z
#define NUM_CHARS 26
#define OFFSET 97

using namespace std;

struct State 
{
    State* next[NUM_CHARS];
    list<string> out;
    State* fail;
};

State* constructStateTrie(const vector<string>& keywords)
{
    State* root = new State();

    // Phase I:
    // Construct the basic trie: follow existing paths of characters, if they do
    // not exist, create them.

    for (auto& it: keywords)
    {
        State* active = root;
        for (int i = 0; i < it.length(); ++i)
        {
            int hash = it[i] - OFFSET;
            if (!active->next[hash])
                active->next[hash] = new State();
            active = active->next[hash]; 
        }
        active->out.push_back(it);
    }

    // Phase II:
    // Construct fail jumps, adjust out function

    queue<State*> q;
    for (int i = 0; i < NUM_CHARS; ++i)
    {
        if (!root->next[i])
            root->next[i] = root;
        else 
        {
            root->next[i]->fail = root;
            q.push(root->next[i]);
        }
    }

    while (!q.empty())
    {
        State* it = q.front();
        q.pop();

        for (int i = 0; i < NUM_CHARS; ++i)
        {
            if (it->next[i])
            {
                State* tmp = it->next[i];
                q.push(tmp);
                State* fail = it->fail;
                while (!fail->next[i]) fail = fail->fail;
                tmp->fail = fail->next[i];
                tmp->out.insert(tmp->out.end(), tmp->fail->out.begin(), tmp->fail->out.end()); 
            }
        }
    }

    return root;
}

// returns a list of matches:
//  A match is a pair of the index the match occured at and a collection of 
//  all the keywords that were matched (note that the index indicates where
//  the keyword(s) ended)
vector<pair<int, vector<string>>> search(State* stateTrie, const string& word)
{
    vector<pair<int, vector<string>>> result;

    for (int i = 0; i < word.length(); ++i)
    {
        int hash = word[i] - OFFSET;
        while (!stateTrie->next[hash]) stateTrie = stateTrie->fail;
        stateTrie = stateTrie->next[hash];
        if (stateTrie->out.size() > 0)
            result.push_back({i, vector<string>(stateTrie->out.begin(), stateTrie->out.end()) });
    }
    return result;
}

int main(int argc, char* args[]) 
{
    // Sample program

    vector<string> keywords = { "this", "is", "the", "end" };
    string text = "athistthend";

    cout << "Keywords: ";
    for (auto& it: keywords)
        cout << it << " ";
    cout << "\nText: " << text << "\n"; 
    
    State* root = constructStateTrie(keywords);
    auto result = search(root, text);

    for (auto& it: result)
    {
        cout << it.first << " ";
        for (auto& it2: it.second)
            cout << it2 << " ";
        cout << "\n";
    }

    return 0;
}
